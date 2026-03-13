import { DemoExperience } from "@/components/demo/demo-experience";
import { ButtonLink } from "@/components/ui/button-link";
import { isDemoReadyForRelativeAssets } from "@/lib/env";

export function LiveDemoSection() {
  const demoReady = isDemoReadyForRelativeAssets();

  return (
    <section className="py-20 sm:py-24">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="section-frame px-6 py-8 sm:px-8 sm:py-10">
          <div className="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
            <div className="section-copy">
              <span className="kicker">Live demo</span>
              <h2 className="mt-5 font-display text-3xl font-semibold tracking-[-0.06em] sm:text-4xl">
                See it work in under two seconds.
              </h2>
              <p className="mt-4 text-lg leading-8 text-muted-foreground">
                Trying the product is more convincing than reading marketing copy.
                Start with a sample, swap products, then turn the same workflow into
                an API integration.
              </p>
            </div>
            <ButtonLink href="/demo" variant="outline" size="lg">
              Open full demo
            </ButtonLink>
          </div>
          <div className="mt-10">
            <DemoExperience compact demoReady={demoReady} />
          </div>
        </div>
      </div>
    </section>
  );
}
