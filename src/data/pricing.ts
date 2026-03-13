export interface PricingTier {
  name: string;
  price: string;
  period: string;
  mockups: string;
  overage: string;
  description: string;
  features: string[];
  cta: string;
  href: string;
  popular?: boolean;
}

export const pricingTiers: PricingTier[] = [
  {
    name: "Free",
    price: "$0",
    period: "/month",
    mockups: "50/month",
    overage: "Included",
    description: "Test quality and start with no credit card.",
    features: [
      "50 mockups each month",
      "5 starter templates",
      "API access",
      "Preview watermark",
      "Single user",
    ],
    cta: "Start free",
    href: "/signup?plan=free",
  },
  {
    name: "Starter",
    price: "$19",
    period: "/month",
    mockups: "500/month",
    overage: "$0.05 each",
    description: "For solo sellers shipping consistent product drops.",
    features: [
      "All 44 templates",
      "Full-resolution exports",
      "Commercial use",
      "Standard queue",
      "Email support",
    ],
    cta: "Start free trial",
    href: "/signup?plan=starter",
  },
  {
    name: "Growth",
    price: "$49",
    period: "/month",
    mockups: "2,000/month",
    overage: "$0.035 each",
    description: "Best for growing stores and automated listing workflows.",
    features: [
      "Batch generation",
      "Webhook support",
      "Priority queue",
      "All templates",
      "Usage analytics",
    ],
    cta: "Start free trial",
    href: "/signup?plan=growth",
    popular: true,
  },
  {
    name: "Pro",
    price: "$99",
    period: "/month",
    mockups: "5,000/month",
    overage: "$0.025 each",
    description: "For teams managing multiple brands or storefronts.",
    features: [
      "Up to 5 team members",
      "Priority rendering",
      "Advanced usage views",
      "Webhook support",
      "Priority support",
    ],
    cta: "Start free trial",
    href: "/signup?plan=pro",
  },
  {
    name: "Platform",
    price: "$299",
    period: "/month",
    mockups: "20,000/month",
    overage: "Custom",
    description: "For platforms, embedded workflows, and reseller rights.",
    features: [
      "Up to 10 team members",
      "Higher rate limits",
      "Multi-key support",
      "SLA-lite support",
      "Custom onboarding",
    ],
    cta: "Contact sales",
    href: "mailto:hello@meetmockup.com?subject=Platform%20plan",
  },
];
