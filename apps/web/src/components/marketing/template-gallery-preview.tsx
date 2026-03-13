import Image from "next/image";

import { templateCategories, totalTemplates } from "@/data/templates";
import { ButtonLink } from "@/components/ui/button-link";
import { Card, CardContent } from "@/components/ui/card";

export function TemplateGalleryPreview() {
  return (
    <section className="py-20 sm:py-24">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="section-frame overflow-hidden px-6 py-8 sm:px-8 sm:py-10">
          <div className="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
            <div className="section-copy">
              <span className="kicker">Template library</span>
              <h2 className="mt-5 font-display text-3xl font-semibold tracking-[-0.06em] sm:text-4xl">
                {totalTemplates} templates across {templateCategories.length} product types
              </h2>
              <p className="mt-4 text-lg leading-8 text-muted-foreground">
                Start with proven categories for POD sellers: tees, hoodies, mugs,
                posters, phone cases, and more.
              </p>
            </div>
            <ButtonLink href="/templates" variant="outline" size="lg">
              Browse all templates
            </ButtonLink>
          </div>
          <div className="mt-10 flex gap-4 overflow-x-auto pb-2">
            {templateCategories.map((category, index) => (
              <Card
                key={category.productType}
                className="min-w-[15.5rem] rounded-[1.8rem] border border-black/8 bg-white/85 py-0 shadow-none"
              >
                <CardContent className="px-4 py-4">
                  <div className="relative aspect-square overflow-hidden rounded-[1.4rem] bg-[#eef2f7]">
                    <Image
                      src={category.thumbnailSrc}
                      alt={category.label}
                      fill
                      className="object-cover"
                      priority={index === 0}
                      sizes="15.5rem"
                    />
                  </div>
                  <div className="mt-4 flex items-end justify-between gap-3">
                    <div>
                      <h3 className="text-base font-semibold text-foreground">
                        {category.label}
                      </h3>
                      <p className="text-sm text-muted-foreground">
                        {category.count} templates
                      </p>
                    </div>
                    <span className="rounded-full bg-muted px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                      {category.productType}
                    </span>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
