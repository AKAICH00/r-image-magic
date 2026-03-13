import type { Metadata } from "next";

import { TemplateGrid } from "@/components/marketing/template-grid";
import { templateCategories, totalTemplates } from "@/data/templates";

export const metadata: Metadata = {
  title: "Templates",
  description: `Browse ${totalTemplates} mockup templates across ${templateCategories.length} product types.`,
};

export default async function TemplatesPage({
  searchParams,
}: {
  searchParams: Promise<{ type?: string }>;
}) {
  const params = await searchParams;

  return (
    <div className="py-16 sm:py-20">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="section-copy">
          <span className="kicker">Template library</span>
          <h1 className="mt-5 font-display text-4xl font-semibold tracking-[-0.06em] text-foreground sm:text-5xl">
            Templates built for real product listings
          </h1>
          <p className="mt-4 text-lg leading-8 text-muted-foreground">
            {totalTemplates} live templates across {templateCategories.length} core
            product types, with more to follow as the catalog expands.
          </p>
        </div>
        <div className="mt-10">
          <TemplateGrid initialType={params.type ?? null} />
        </div>
      </div>
    </div>
  );
}
