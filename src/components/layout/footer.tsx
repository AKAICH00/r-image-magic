import Link from "next/link";

import { SiteLogo } from "@/components/layout/site-logo";

const footerSections = [
  {
    title: "Product",
    links: [
      { href: "/demo", label: "Demo" },
      { href: "/templates", label: "Templates" },
      { href: "/pricing", label: "Pricing" },
    ],
  },
  {
    title: "Developers",
    links: [
      { href: "/docs", label: "API Docs" },
      { href: "/docs/api-reference", label: "API Reference" },
      { href: "https://api.meetmockup.com/swagger-ui/", label: "Swagger UI", external: true },
    ],
  },
  {
    title: "Company",
    links: [
      { href: "mailto:hello@meetmockup.com", label: "Contact", external: true },
      { href: "/demo", label: "Try the demo" },
    ],
  },
  {
    title: "Legal",
    links: [
      { href: "/terms", label: "Terms" },
      { href: "/privacy", label: "Privacy" },
    ],
  },
];

export function Footer() {
  return (
    <footer className="relative overflow-hidden border-t border-black/8 bg-[linear-gradient(180deg,#f7f4ec_0%,#f2ede3_100%)]">
      <div className="absolute inset-x-0 top-0 h-px bg-[linear-gradient(90deg,transparent,var(--brand-signal),transparent)] opacity-60" />
      <div className="mx-auto grid max-w-7xl gap-12 px-4 py-14 sm:px-6 lg:grid-cols-[1.2fr_repeat(4,1fr)] lg:px-8">
        <div className="max-w-sm space-y-4">
          <SiteLogo />
          <p className="text-sm leading-6 text-muted-foreground">
            Generate photorealistic mockups with displacement mapping, fast enough
            for product launches and clean enough for API-first workflows.
          </p>
        </div>
        {footerSections.map((section) => (
          <div key={section.title}>
            <h3 className="text-sm font-semibold tracking-[0.12em] text-foreground/80 uppercase">
              {section.title}
            </h3>
            <ul className="mt-4 space-y-3">
              {section.links.map((link) => (
                <li key={link.label}>
                  <Link
                    href={link.href}
                    target={link.external ? "_blank" : undefined}
                    rel={link.external ? "noreferrer" : undefined}
                    className="text-sm text-muted-foreground transition-colors hover:text-foreground"
                  >
                    {link.label}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <div className="mx-auto flex max-w-7xl items-center justify-between border-t border-black/8 px-4 py-6 text-sm text-muted-foreground sm:px-6 lg:px-8">
        <span>&copy; 2026 MeetMockup. All rights reserved.</span>
        <span>Built for POD sellers and developers.</span>
      </div>
    </footer>
  );
}
