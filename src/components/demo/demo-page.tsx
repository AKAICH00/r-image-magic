import { DemoExperience } from "@/components/demo/demo-experience";
import { isDemoReadyForRelativeAssets } from "@/lib/env";

export function DemoPage() {
  const demoReady = isDemoReadyForRelativeAssets();

  return (
    <div className="py-16 sm:py-20">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="section-copy">
          <span className="kicker">Interactive demo</span>
          <h1 className="mt-5 font-display text-4xl font-semibold tracking-[-0.06em] text-foreground sm:text-5xl">
            Try MeetMockup with a real product engine
          </h1>
          <p className="mt-4 text-lg leading-8 text-muted-foreground">
            Use one of the sample graphics or upload your own design. The demo
            proxies through the same generation API used for production workflows.
          </p>
        </div>
        <div className="mt-10">
          <DemoExperience demoReady={demoReady} />
        </div>
      </div>
    </div>
  );
}
