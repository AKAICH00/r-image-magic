import Link from "next/link";

import { cn } from "@/lib/utils";

interface SiteLogoProps {
  className?: string;
}

export function SiteLogo({ className }: SiteLogoProps) {
  return (
    <Link href="/" className={cn("inline-flex items-center gap-3", className)}>
      <span className="relative flex size-9 items-center justify-center overflow-hidden rounded-2xl bg-[linear-gradient(135deg,var(--brand-signal),var(--brand-sun))] text-[0.7rem] font-semibold tracking-[0.2em] text-[#0f1218] shadow-[0_14px_40px_rgba(255,173,79,0.28)]">
        MM
        <span className="absolute inset-0 rounded-2xl ring-1 ring-black/10" />
      </span>
      <span className="flex flex-col leading-none">
        <span className="font-display text-[1rem] font-semibold tracking-[-0.04em] text-foreground">
          MeetMockup
        </span>
        <span className="text-[0.64rem] uppercase tracking-[0.24em] text-muted-foreground">
          Realistic product mockups
        </span>
      </span>
    </Link>
  );
}
