import { DownloadIcon, Layers3Icon, UploadIcon } from "lucide-react";

import { Card, CardContent } from "@/components/ui/card";

const steps = [
  {
    icon: UploadIcon,
    title: "Upload your design",
    description: "PNG or JPG. Transparent backgrounds work best.",
  },
  {
    icon: Layers3Icon,
    title: "Pick a product",
    description: "T-shirts, hoodies, mugs, phone cases, and more. 44 templates live now.",
  },
  {
    icon: DownloadIcon,
    title: "Get a real mockup",
    description: "Download it instantly or automate the same flow through one API call.",
  },
];

export function HowItWorks() {
  return (
    <section className="py-20 sm:py-24">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="section-frame px-6 py-8 sm:px-8 sm:py-10">
          <div className="section-copy">
            <span className="kicker">How it works</span>
            <h2 className="mt-5 font-display text-3xl font-semibold tracking-[-0.06em] sm:text-4xl">
              Three steps. That&apos;s it.
            </h2>
          </div>
          <div className="mt-10 grid gap-4 lg:grid-cols-3">
            {steps.map((step, index) => {
              const Icon = step.icon;

              return (
                <Card
                  key={step.title}
                  className="rounded-[1.8rem] border border-black/8 bg-white/78 py-0 shadow-none"
                >
                  <CardContent className="px-6 py-6">
                    <div className="flex items-center justify-between">
                      <div className="flex size-12 items-center justify-center rounded-2xl bg-[linear-gradient(135deg,var(--brand-signal),var(--brand-sun))] text-[#151b26] shadow-lg">
                        <Icon className="size-5" />
                      </div>
                      <span className="font-display text-3xl text-[#151b26]/18">
                        0{index + 1}
                      </span>
                    </div>
                    <h3 className="mt-8 text-xl font-semibold text-foreground">
                      {step.title}
                    </h3>
                    <p className="mt-3 leading-7 text-muted-foreground">
                      {step.description}
                    </p>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </div>
      </div>
    </section>
  );
}
