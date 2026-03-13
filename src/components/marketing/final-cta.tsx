import { ButtonLink } from "@/components/ui/button-link";

export function FinalCta() {
  return (
    <section className="pb-20 pt-12 sm:pb-24">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="overflow-hidden rounded-[2.2rem] border border-white/8 bg-[#0f1218] px-6 py-12 text-white shadow-[0_28px_110px_rgba(15,18,24,0.24)] sm:px-10">
          <div className="grid gap-6 lg:grid-cols-[1fr_auto] lg:items-center">
            <div>
              <span className="kicker border-white/10 bg-white/5 text-white/64">
                Ready to ship better listings
              </span>
              <h2 className="mt-5 max-w-2xl font-display text-3xl font-semibold tracking-[-0.06em] sm:text-4xl">
                Your listings deserve better mockups.
              </h2>
              <p className="mt-4 max-w-2xl text-lg leading-8 text-white/64">
                Try the interactive demo now, then upgrade into an API workflow
                when you need repeatable output at scale.
              </p>
            </div>
            <ButtonLink
              href="/signup"
              size="lg"
              className="h-11 bg-[linear-gradient(135deg,var(--brand-signal),var(--brand-sun))] px-5 text-[#11151f] hover:opacity-90"
            >
              Start generating it&apos;s free
            </ButtonLink>
          </div>
        </div>
      </div>
    </section>
  );
}
