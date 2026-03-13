import { ArrowRightIcon } from "lucide-react";

import { HeroRotator } from "@/components/marketing/hero-rotator";
import { ButtonLink } from "@/components/ui/button-link";

const stats = [
  { value: "44", label: "templates live" },
  { value: "9", label: "product types" },
  { value: "~1.5s", label: "typical generation" },
];

export function Hero() {
  return (
    <section className="mesh-highlight relative overflow-hidden">
      <div className="absolute inset-0 bg-[linear-gradient(135deg,rgba(255,255,255,0.03),transparent_40%,rgba(217,235,255,0.1))]" />
      <div className="mx-auto grid max-w-7xl gap-14 px-4 py-18 sm:px-6 sm:py-24 lg:grid-cols-[1.05fr_0.95fr] lg:px-8 lg:py-28">
        <div className="relative z-10 flex flex-col justify-center">
          <span className="kicker border-white/12 bg-white/8 text-white/68">
            Linear-grade polish. Replicate-grade utility.
          </span>
          <h1 className="mt-7 max-w-xl font-display text-5xl font-semibold tracking-[-0.07em] text-white sm:text-6xl lg:text-7xl">
            Mockups that look real.
          </h1>
          <p className="mt-6 max-w-xl text-lg leading-8 text-white/68 sm:text-xl">
            MeetMockup uses displacement mapping to wrap your designs onto
            products with real fabric texture, curved surfaces, and subtle wear,
            not flat overlays pasted on top.
          </p>
          <div className="mt-9 flex flex-col gap-3 sm:flex-row">
            <ButtonLink
              href="/demo"
              size="lg"
              className="h-11 bg-[linear-gradient(135deg,var(--brand-signal),var(--brand-sun))] px-5 text-[#11151f] hover:opacity-90"
            >
              Try it free
            </ButtonLink>
            <ButtonLink
              href="/docs"
              variant="outline"
              size="lg"
              className="h-11 border-white/14 bg-white/5 px-5 text-white hover:bg-white/10 hover:text-white"
            >
              View API docs
              <ArrowRightIcon className="size-4" />
            </ButtonLink>
          </div>
          <div className="mt-10 grid max-w-xl gap-3 sm:grid-cols-3">
            {stats.map((stat) => (
              <div
                key={stat.label}
                className="rounded-3xl border border-white/10 bg-white/6 px-4 py-4 backdrop-blur"
              >
                <div className="font-display text-2xl font-semibold tracking-[-0.06em] text-white">
                  {stat.value}
                </div>
                <div className="mt-1 text-sm text-white/55">{stat.label}</div>
              </div>
            ))}
          </div>
        </div>
        <div className="relative z-10 flex items-center justify-center">
          <HeroRotator />
        </div>
      </div>
    </section>
  );
}
