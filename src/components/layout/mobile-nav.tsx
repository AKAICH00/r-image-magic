"use client";

import { UserButton, useUser } from "@clerk/nextjs";
import Link from "next/link";
import { MenuIcon } from "lucide-react";

import { ButtonLink } from "@/components/ui/button-link";
import { buttonVariants } from "@/components/ui/button-styles";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { SiteLogo } from "@/components/layout/site-logo";
import { cn } from "@/lib/utils";

const navLinks = [
  { href: "/demo", label: "Demo" },
  { href: "/templates", label: "Templates" },
  { href: "/pricing", label: "Pricing" },
  { href: "/docs", label: "Docs" },
];

interface MobileNavProps {
  withAuth?: boolean;
}

export function MobileNav({ withAuth = true }: MobileNavProps) {
  const clerkEnabled = Boolean(process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY);

  return (
    <Sheet>
      <SheetTrigger
        className={cn(
          buttonVariants({ variant: "ghost", size: "icon-sm" }),
          "md:hidden",
        )}
      >
        <MenuIcon className="size-4" />
        <span className="sr-only">Open navigation</span>
      </SheetTrigger>
      <SheetContent
        side="right"
        className="w-[min(88vw,24rem)] border-l border-white/10 bg-[#0f1218] text-white"
      >
        <SheetHeader className="border-b border-white/10 pb-5">
          <SiteLogo className="[&_span:last-child_span:first-child]:text-white [&_span:last-child_span:last-child]:text-white/60" />
          <SheetTitle className="sr-only">Navigation</SheetTitle>
          <SheetDescription className="text-white/60">
            Explore the demo, templates, pricing, and API docs.
          </SheetDescription>
        </SheetHeader>
        <nav className="flex flex-col gap-2 p-4">
          {navLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className="rounded-2xl border border-white/10 px-4 py-3 text-sm font-medium text-white/88 transition hover:border-[var(--brand-signal)] hover:bg-white/5"
            >
              {link.label}
            </Link>
          ))}
        </nav>
        {withAuth ? (
          <div className="mt-auto flex flex-col gap-3 p-4">
            {clerkEnabled ? (
              <ClerkMobileAuth />
            ) : (
              <>
                <ButtonLink
                  href="/login"
                  variant="outline"
                  className="border-white/15 bg-white/5 text-white hover:bg-white/10"
                >
                  Log in
                </ButtonLink>
                <ButtonLink
                  href="/signup"
                  className="bg-[linear-gradient(135deg,var(--brand-signal),var(--brand-sun))] text-[#11151f] hover:opacity-90"
                >
                  Sign up free
                </ButtonLink>
              </>
            )}
          </div>
        ) : null}
      </SheetContent>
    </Sheet>
  );
}

function ClerkMobileAuth() {
  const { isLoaded, isSignedIn } = useUser();

  if (!isLoaded) {
    return null;
  }

  if (!isSignedIn) {
    return (
      <>
        <ButtonLink
          href="/login"
          variant="outline"
          className="border-white/15 bg-white/5 text-white hover:bg-white/10"
        >
          Log in
        </ButtonLink>
        <ButtonLink
          href="/signup"
          className="bg-[linear-gradient(135deg,var(--brand-signal),var(--brand-sun))] text-[#11151f] hover:opacity-90"
        >
          Sign up free
        </ButtonLink>
      </>
    );
  }

  return (
    <>
      <Link
        href="/dashboard"
        className="inline-flex h-8 items-center justify-center rounded-lg border border-white/15 bg-white/5 px-3 text-sm font-medium text-white transition hover:bg-white/10"
      >
        Dashboard
      </Link>
      <div className="flex justify-center pt-2">
        <UserButton />
      </div>
    </>
  );
}
