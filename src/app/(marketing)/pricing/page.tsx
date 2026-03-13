import type { Metadata } from "next";

import { PricingTable } from "@/components/marketing/pricing-table";

export const metadata: Metadata = {
  title: "Pricing",
  description:
    "Transparent pricing for realistic product mockups. Start free, then scale into higher-volume plans.",
};

export default function PricingPage() {
  return (
    <div className="py-16 sm:py-20">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="section-copy">
          <span className="kicker">Pricing</span>
          <h1 className="mt-5 font-display text-4xl font-semibold tracking-[-0.06em] text-foreground sm:text-5xl">
            Simple, transparent pricing
          </h1>
          <p className="mt-4 text-lg leading-8 text-muted-foreground">
            Start free with 50 mockups per month. Upgrade when you need more
            templates, cleaner exports, higher throughput, or team access.
          </p>
        </div>
        <div className="mt-10">
          <PricingTable />
        </div>
        <p className="mt-10 text-sm text-muted-foreground">
          Need custom volume, SLAs, or dedicated infrastructure?{" "}
          <a
            href="mailto:hello@meetmockup.com?subject=Enterprise%20inquiry"
            className="font-medium text-foreground underline underline-offset-4"
          >
            Contact us.
          </a>
        </p>
      </div>
    </div>
  );
}
