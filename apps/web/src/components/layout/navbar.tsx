import Link from "next/link";

import { MobileNav } from "@/components/layout/mobile-nav";
import { NavbarAuth } from "@/components/layout/navbar-auth";
import { SiteLogo } from "@/components/layout/site-logo";
import { ButtonLink } from "@/components/ui/button-link";
import { isClerkConfigured } from "@/lib/env";

const navLinks = [
  { href: "/demo", label: "Demo" },
  { href: "/templates", label: "Templates" },
  { href: "/pricing", label: "Pricing" },
  { href: "/docs", label: "Docs" },
];

export function Navbar() {
  const clerkEnabled = isClerkConfigured();

  return (
    <header className="sticky top-0 z-50 border-b border-black/8 bg-[color:rgba(247,244,236,0.88)] backdrop-blur-xl">
      <div className="mx-auto flex h-18 max-w-7xl items-center justify-between gap-5 px-4 sm:px-6 lg:px-8">
        <div className="flex items-center gap-8">
          <SiteLogo />
          <nav className="hidden items-center gap-1 md:flex">
            {navLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className="rounded-full px-3 py-2 text-sm font-medium text-muted-foreground transition hover:bg-white/80 hover:text-foreground"
              >
                {link.label}
              </Link>
            ))}
          </nav>
        </div>
        <div className="flex items-center gap-3">
          {clerkEnabled ? (
            <NavbarAuth />
          ) : (
            <div className="hidden items-center gap-3 md:flex">
              <ButtonLink href="/login" variant="ghost" size="sm">
                Log in
              </ButtonLink>
              <ButtonLink
                href="/signup"
                size="sm"
                className="bg-[linear-gradient(135deg,var(--brand-signal),var(--brand-sun))] text-[#11151f] hover:opacity-90"
              >
                Sign up free
              </ButtonLink>
            </div>
          )}
          <MobileNav withAuth />
        </div>
      </div>
    </header>
  );
}
