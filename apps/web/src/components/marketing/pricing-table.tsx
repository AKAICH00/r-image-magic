import { CheckIcon } from "lucide-react";

import { pricingTiers } from "@/data/pricing";
import { Badge } from "@/components/ui/badge";
import { ButtonLink } from "@/components/ui/button-link";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export function PricingTable() {
  return (
    <div className="grid gap-5 xl:grid-cols-5 lg:grid-cols-3 md:grid-cols-2">
      {pricingTiers.map((tier) => (
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
          <CardContent className="flex h-full flex-col px-6 pb-6">
            <div className="rounded-[1.2rem] bg-muted px-4 py-3 text-sm font-medium text-foreground">
              {tier.mockups} · {tier.overage}
            </div>
            <ul className="mt-5 flex-1 space-y-3">
              {tier.features.map((feature) => (
                <li
                  key={feature}
                  className="flex items-start gap-3 text-sm leading-6 text-muted-foreground"
                >
                  <CheckIcon className="mt-0.5 size-4 shrink-0 text-[#151b26]" />
                  <span>{feature}</span>
                </li>
              ))}
            </ul>
            <div className="mt-6">
              <ButtonLink
                href={tier.href}
                variant={tier.popular ? "default" : "outline"}
                className={tier.popular ? "w-full" : "w-full"}
              >
                {tier.cta}
              </ButtonLink>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
