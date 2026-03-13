import { pricingTiers } from "@/data/pricing";
import { Badge } from "@/components/ui/badge";
import { ButtonLink } from "@/components/ui/button-link";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

const teaserTiers = pricingTiers.filter((tier) =>
  ["Free", "Growth", "Pro"].includes(tier.name),
);

export function PricingTeaser() {
  return (
    <section className="py-20 sm:py-24">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="section-frame px-6 py-8 sm:px-8 sm:py-10">
          <div className="section-copy">
            <span className="kicker">Pricing</span>
            <h2 className="mt-5 font-display text-3xl font-semibold tracking-[-0.06em] sm:text-4xl">
              Start free. Scale when you&apos;re ready.
            </h2>
            <p className="mt-4 text-lg leading-8 text-muted-foreground">
              Start with 50 free mockups per month. No credit card required.
              Upgrade when you need more volume, batch generation, or team access.
            </p>
          </div>
          <div className="mt-10 grid gap-4 lg:grid-cols-3">
            {teaserTiers.map((tier) => (
              <Card
                key={tier.name}
                className={`rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none ${
                  tier.popular ? "relative ring-2 ring-[#151b26]" : ""
                }`}
              >
                {tier.popular ? (
                  <Badge className="absolute right-4 top-4 bg-[linear-gradient(135deg,var(--brand-signal),var(--brand-sun))] text-[#11151f]">
                    Most popular
                  </Badge>
                ) : null}
                <CardHeader className="gap-2 px-6 py-6">
                  <CardTitle className="text-xl">{tier.name}</CardTitle>
                  <div className="font-display text-5xl font-semibold tracking-[-0.07em] text-foreground">
                    {tier.price}
                    <span className="ml-2 text-lg font-medium text-muted-foreground">
                      {tier.period}
                    </span>
                  </div>
                  <p className="text-sm leading-6 text-muted-foreground">
                    {tier.description}
                  </p>
                </CardHeader>
                <CardContent className="px-6 pb-6">
                  <div className="rounded-[1.2rem] bg-muted px-4 py-3 text-sm font-medium text-foreground">
                    {tier.mockups}
                  </div>
                  <p className="mt-4 text-sm text-muted-foreground">
                    {tier.features.slice(0, 3).join(" · ")}
                  </p>
                </CardContent>
              </Card>
            ))}
          </div>
          <div className="mt-8">
            <ButtonLink href="/pricing" variant="outline" size="lg">
              See all plans
            </ButtonLink>
          </div>
        </div>
      </div>
    </section>
  );
}
